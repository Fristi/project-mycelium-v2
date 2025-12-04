package co.mycelium.domain

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
)  derives Encoder.AsObject, Decoder
