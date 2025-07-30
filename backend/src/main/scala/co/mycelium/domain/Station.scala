package co.mycelium.domain

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
)
