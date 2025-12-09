package co.mycelium.domain

import scala.concurrent.duration.FiniteDuration
import io.circe.*
import io.circe.syntax.*
import io.circe.parser.*

sealed trait StationEvent

object StationEvent:
  case class Watered(period: FiniteDuration) extends StationEvent

  given Encoder.AsObject[StationEvent] = Encoder.AsObject.derived
  given Decoder[StationEvent] = Decoder.derived
