package co.mycelium.domain

import io.circe.{Decoder, Encoder}

import java.time.Instant
import java.util.UUID

final case class StationLog(stationId: UUID, on: Instant, event: StationEvent) derives Encoder.AsObject, Decoder
