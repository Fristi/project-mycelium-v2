package co.mycelium

import blobstore.s3.S3Store
import cats.data.Kleisli
import cats.effect.*
import cats.effect.std.UUIDGen
import cats.implicits.*
import co.mycelium.{AppConfig, Database}
import co.mycelium.endpoints.Stations
import co.mycelium.service.{StationService, StationServiceImpl}
import co.mycelium.ports.*
import com.comcast.ip4s.*
import org.http4s.ember.server.EmberServerBuilder
import org.http4s.server.middleware.{ErrorAction, ErrorHandling}
import org.http4s.server.staticcontent.*
import org.http4s.server.{Router, Server}
import org.http4s.{HttpApp, Request, Response}
import org.typelevel.log4cats.slf4j.Slf4jFactory
import org.typelevel.log4cats.{Logger, LoggerFactory}
import software.amazon.awssdk.auth.credentials.{AwsBasicCredentials, StaticCredentialsProvider}
import software.amazon.awssdk.core.client.config.ClientOverrideConfiguration
import software.amazon.awssdk.core.retry.RetryPolicy
import software.amazon.awssdk.core.retry.conditions.RetryCondition
import software.amazon.awssdk.http.async.SdkAsyncHttpClient
import software.amazon.awssdk.http.nio.netty.NettyNioAsyncHttpClient
import software.amazon.awssdk.regions.Region
import software.amazon.awssdk.services.s3.S3AsyncClient
import software.amazon.awssdk.services.s3.model.{BucketAlreadyExistsException, CreateBucketRequest}

import java.net.URI
import java.time.Duration

object Main extends IOApp {

  implicit val loggerFactory: LoggerFactory[IO] = Slf4jFactory.create[IO]

  override def run(args: List[String]): IO[ExitCode] =
    app.use(_ => IO.never).as(ExitCode.Success)

  def httpApp(svc: StationService[IO]): HttpApp[IO] =
    Router("api" -> Stations.routes(svc)).orNotFound

  private def errorHandling(log: Logger[IO])(route: Kleisli[IO, Request[IO], Response[IO]]) =
    ErrorHandling.Recover.total(
      ErrorAction.log(route, (t, msg) => log.warn(t)(msg), (t, msg) => log.error(t)(msg))
    )

  private def stationService(cfg: AppConfig): Resource[IO, StationService[IO]] = for {
    tx              <- Database.transactor[IO](cfg.db, loggerFactory.getLoggerFromName("Doobie"))
    plantClassifier <- PlantClassifier.fromConfig[IO](cfg.plantClassifier)
    plantProfiler   <- PlantProfiler.fromConfig[IO](cfg.plantProfiler)
    s3Client = client(cfg.blob)
    _  <- Resource.eval(createBucket(s3Client))
    s3 <- Resource.eval(
      IO.fromOption(S3Store.builder[IO](s3Client).build.toOption)(
        new Throwable("Unable to build s3 client")
      )
    )
  } yield new StationServiceImpl[IO](
    uuidGen = UUIDGen[IO],
    clock = Clock[IO],
    repos = Repositories.fromTransactor(tx),
    s3 = s3,
    plantClassifier = plantClassifier,
    plantProfiler = plantProfiler,
    plantAvatarPlaceHolder = new ResourcePlantAvatarPlaceHolder[IO]
  )

  val app: Resource[IO, Server] =
    for {
      cfg <- Resource.eval(AppConfig.config.load[IO])
      _   <- Resource.eval(Database.flyway[IO](cfg.db))
      svc <- stationService(cfg)
      errorLogging = loggerFactory.getLoggerFromName("Http4s")
      app          = errorHandling(errorLogging)(httpApp(svc))
      server <- EmberServerBuilder
        .default[IO]
        .withHost(ipv4"0.0.0.0")
        .withPort(port"8080")
        .withHttpApp(app)
        .build
    } yield server

  private val overrideConfiguration: ClientOverrideConfiguration =
    ClientOverrideConfiguration
      .builder()
      .apiCallTimeout(Duration.ofSeconds(30))
      .apiCallAttemptTimeout(Duration.ofSeconds(20))
      .retryPolicy(
        RetryPolicy
          .builder()
          .numRetries(5)
          .retryCondition(RetryCondition.defaultRetryCondition())
          .build()
      )
      .build()

  private val httpClient: SdkAsyncHttpClient = NettyNioAsyncHttpClient
    .builder()
    .connectionTimeout(Duration.ofSeconds(20))
    .connectionAcquisitionTimeout(Duration.ofSeconds(20))
    .connectionMaxIdleTime(Duration.ofSeconds(10))
    .build()

  private def client(blobConfig: S3BlobConfig): S3AsyncClient = S3AsyncClient
    .builder()
    .region(Region.US_EAST_1)
    .credentialsProvider(
      StaticCredentialsProvider.create(
        AwsBasicCredentials.create(blobConfig.accessKey, blobConfig.secretKey.value)
      )
    )
    .endpointOverride(URI.create(blobConfig.host))
    .overrideConfiguration(overrideConfiguration)
    .forcePathStyle(true)
    .httpClient(httpClient)
    .build()

  private def createBucket(s3: S3AsyncClient) =
    IO.fromCompletableFuture(
      IO.delay(s3.createBucket(CreateBucketRequest.builder().bucket("mycelium").build()))
    ).void
      .recoverWith {
        case _: BucketAlreadyExistsException => IO.unit
        case error                           => IO.unit
      }
}
