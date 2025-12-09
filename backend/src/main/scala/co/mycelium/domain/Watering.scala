package co.mycelium.domain

import io.circe.{Decoder, Encoder}

import scala.concurrent.duration.FiniteDuration

final case class Watering(watering: Option[FiniteDuration])  derives Encoder.AsObject, Decoder
