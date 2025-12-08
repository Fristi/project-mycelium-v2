package co.mycelium.endpoints

import io.circe.{Decoder, Encoder}

final case class MyceliumError(error: String) derives Encoder.AsObject, Decoder
