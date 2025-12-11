package co.mycelium.domain

import io.circe.{Decoder, Encoder}

import java.time.Instant
import java.util.UUID

final case class StationInsert(
    mac: String,
    name: String
) derives Encoder.AsObject,
      Decoder {
  def toStation(id: UUID, created: Instant, userId: String): Station =
    Station(
      id = id,
      mac = mac,
      name = name,
      location = None,
      description = None,
      userId = userId,
      created = created,
      updated = None
    )
}
