package co.mycelium.endpoints

import cats.effect.IO
import co.mycelium.db.Repositories
import co.mycelium.domain.*
import co.mycelium.service.StationService
import cron4s.CronExpr
import cron4s.lib.javatime.javaTemporalInstance
import org.http4s.HttpRoutes
import sttp.tapir.*
import sttp.tapir.generic.Configuration
import sttp.tapir.generic.auto.*
import sttp.tapir.json.circe.*
import sttp.tapir.server.http4s.Http4sServerInterpreter

import java.time.Instant
import java.util.UUID
import scala.concurrent.duration.FiniteDuration
import sttp.tapir.server.http4s.Http4sServerOptions
import sttp.tapir.server.model.ValuedEndpointOutput
import sttp.tapir.server.interceptor.log.ServerLog
import sttp.tapir.server.interceptor.log.DefaultServerLog
import sttp.tapir.server.interceptor.cors.CORSInterceptor
import sttp.tapir.server.interceptor.cors.CORSConfig
import sttp.model.headers.Origin

import scala.annotation.experimental

@experimental
object Stations extends TapirSchemas {

  object endpoints {
    val stations = base.in("stations")

    val list    = stations.get.out(jsonBody[List[Station]]).name("listStations")
    val add     = stations.post.in(jsonBody[StationInsert]).out(jsonBody[UUID]).name("addStation")
    val details = stations.get
      .in(path[UUID]("stationId"))
      .in(query[Option[MeasurementPeriod]]("period"))
      .name("getStation")
      .out(jsonBody[StationDetails])
    val update =
      stations.put.in(path[UUID]("stationId")).in(jsonBody[StationUpdate]).name("updateStation")
    val delete  = stations.delete.in(path[UUID]("stationId")).name("deleteStation")
    val checkIn = stations
      .in(path[UUID]("stationId"))
      .in("checkin")
      .put
      .in(jsonBody[List[StationMeasurement]])
      .name("checkinStation")
      .out(jsonBody[Watering])
    val watered = stations
      .in(path[UUID]("stationId"))
      .in("watered")
      .post
      .in(jsonBody[Watering])
      .name("wateredAtStation")
    val log = stations
      .in(path[UUID]("stationId"))
      .in("log")
      .in(query[Option[Long]]("page"))
      .name("getStationLog")
      .out(jsonBody[List[StationLog]])

    val all = Set(list, add, details, update, delete, checkIn, watered, log)
  }

  def toMyceliumError(error: String): ValuedEndpointOutput[?] =
    ValuedEndpointOutput(jsonBody[MyceliumError], MyceliumError(error))

  val serverOptions = Http4sServerOptions
    .customiseInterceptors[IO]
    .defaultHandlers(toMyceliumError)
    .serverLog(
      Some(
        Http4sServerOptions
          .defaultServerLog[IO]
          .logWhenHandled(true)
          .logWhenReceived(true)
          .logAllDecodeFailures(true)
      )
    )
    .corsInterceptor(
      CORSInterceptor.customOrThrow[IO](
        CORSConfig.default.allowCredentials
          .allowOrigin(Origin.Host("http", "localhost", Some(1420)))
      )
    )
    .options

  def routes(svc: StationService[IO]): HttpRoutes[IO] = {

    val list = endpoints.list.serverLogic(at => _ => svc.list(at.sub).map(Right(_)))
    val add = endpoints.add.serverLogic(at => insert => svc.add(at.sub, insert).map(Right(_)))
    val delete = endpoints.delete.serverLogic(at => id => svc.delete(at.sub, id).as(Right(())))
    val checkin = endpoints.checkIn.serverLogic(at => (id, measurements) => svc.checkin(at.sub, id, measurements).map(Right(_)))
    val update = endpoints.update.serverLogic(at => (id, update) => svc.update(at.sub, id, update).map(Right(_)))
    val details = endpoints.details.serverLogic(at => (id, period) => svc.details(at.sub, period, id))
    val watered = endpoints.watered.serverLogic(at => (id, request) => svc.watered(at.sub, id, request).map(Right(_)))
    val log = endpoints.log.serverLogic(at => (id, page) => svc.getLogs(at.sub, id, page).map(Right(_)))

    Http4sServerInterpreter(serverOptions)
      .toRoutes(List(list, add, delete, log, watered, checkin, details, update))
  }
}

trait TapirSchemas {
  implicit val customConfiguration: Configuration =
    Configuration.default.withDiscriminator("_type")

  implicit val schemaCronExpr: Schema[CronExpr]             = Schema.string
  implicit val schemaFiniteDuration: Schema[FiniteDuration] = Schema.string

  implicit val codecMeasurementPeriod: Codec[String, MeasurementPeriod, CodecFormat.TextPlain] =
    Codec.string.map(
      Mapping.fromDecode((str: String) =>
        DecodeResult.fromOption(MeasurementPeriod.fromString(str))
      )(_.repr)
    )
}
