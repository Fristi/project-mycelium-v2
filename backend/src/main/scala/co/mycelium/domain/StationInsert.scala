package co.mycelium.domain

import java.time.Instant
import java.util.UUID

final case class StationInsert(
    mac: String,
    name: String
) {
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
