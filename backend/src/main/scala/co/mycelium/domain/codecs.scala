package co.mycelium.domain

import co.mycelium.domain.*
import co.mycelium.endpoints.MyceliumError
import cron4s.{Cron, CronExpr}
import io.circe.generic.semiauto.deriveCodec
import io.circe.{Codec, Decoder, Encoder}

import scala.concurrent.duration.{Duration, FiniteDuration}

given Codec[FiniteDuration] = Codec.from(
  Decoder.decodeString.emap { s =>
    Duration(s) match {
      case fd: FiniteDuration => Right(fd)
      case _ => Left(s"Invalid FiniteDuration: $s")
    }
  },
  Encoder.encodeString.contramap(_.toString)
)

given Codec[CronExpr] = Codec.from(
  Decoder.decodeString.emap(s => Cron.parse(s).left.map(_.getMessage)),
  Encoder.encodeString.contramap(_.toString)
)

given Encoder[FiniteDuration] = Encoder.encodeString.contramap(_.toString)

given Decoder[FiniteDuration] =
  Decoder.decodeString.emap { str =>
    Duration(str) match {
      case fd: FiniteDuration => Right(fd)
      case _                  => Left(s"Invalid FiniteDuration: $str")
    }
  }