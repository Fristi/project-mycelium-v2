package co.mycelium.domain

import io.circe.{Decoder, Encoder}

final case class StationDetails(station: Station, measurements: List[StationMeasurement]) derives Encoder.AsObject, Decoder
