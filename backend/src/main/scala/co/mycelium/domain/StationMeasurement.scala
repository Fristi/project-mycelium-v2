package co.mycelium.domain

import cats.Eq
import cats.derived.*
import org.typelevel.cats.time.*
import io.circe.{Decoder, Encoder}

import java.time.Instant

final case class StationMeasurement(
    on: Instant,
    batteryVoltage: Double,
    temperature: Double,
    humidity: Double,
    lux: Double,
    soilPf: Double,
    tankPf: Double
) derives Encoder.AsObject,
      Decoder,
      Eq
