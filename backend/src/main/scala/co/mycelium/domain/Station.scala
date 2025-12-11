package co.mycelium.domain

import io.circe.{Decoder, Encoder}

import java.time.Instant
import java.util.UUID

final case class Station(
    id: UUID,
    mac: String,
    name: String,
    location: Option[String],
    description: Option[String],
    userId: String,
    created: Instant,
    updated: Option[Instant]
) derives Encoder.AsObject,
      Decoder
