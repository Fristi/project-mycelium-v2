package co.mycelium.endpoints

import cats.data.EitherT
import cats.effect.IO
import com.auth0.jwk.JwkProviderBuilder
import io.circe.Decoder
import io.circe.generic.semiauto.deriveDecoder
import pdi.jwt._

import java.util.concurrent.TimeUnit
import scala.util.Try

object Auth {
  final case class AccessToken(sub: String)

  object AccessToken {
    implicit val decoder: Decoder[AccessToken] = deriveDecoder
  }

  val jwkUrl = sys.env.get("AUTH0_BASE_URL").getOrElse("https://mycelium-green.eu.auth0.com")
  val jwkProvider = new JwkProviderBuilder(jwkUrl)
    .cached(3600, 3600, TimeUnit.SECONDS)
    .build()

  def validate(jwt: String): IO[Either[Unit, AccessToken]] = jwt match {
    case s"$header.$_.$_" =>
      (for {
        jwtHeader <- EitherT.fromOption[IO](
          Try(JwtCirce.parseHeader(JwtBase64.decodeString(header))).toOption,
          ()
        )
        kid <- EitherT.fromOption[IO](jwtHeader.keyId, ())
        jwk <- EitherT.fromOption[IO](Try(jwkProvider.get(kid)).toOption, ())
        a   <- EitherT.fromOption[IO](Jwt.decodeAll(jwt, jwk.getPublicKey).toOption, ())
        token <- EitherT.fromOption[IO](
          io.circe.parser.decode[AccessToken](a._2.content).toOption,
          ()
        )
      } yield token).value

    case _ =>
      IO(Left(()))
  }
}
