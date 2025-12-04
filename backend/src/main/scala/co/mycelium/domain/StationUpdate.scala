package co.mycelium.domain

import io.circe.{Decoder, Encoder}

final case class StationUpdate(
    name: Option[String] = None,
    location: Option[String] = None,
    description: Option[String] = None
)  derives Encoder.AsObject, Decoder
