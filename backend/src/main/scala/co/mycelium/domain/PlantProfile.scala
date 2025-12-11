package co.mycelium.domain

import io.circe.{Decoder, Encoder}

case class Interval[A](start: A, end: A)

object Interval {
  given [A: Encoder]: Encoder.AsObject[Interval[A]] =
    Encoder.AsObject.derived

  given [A: Decoder]: Decoder[Interval[A]] =
    Decoder.derived
}

case class StationPlantProfile(stationId: String, mac: String, profile: PlantProfile)
    derives Encoder.AsObject,
      Decoder

case class PlantProfile(
    name: String,
    variables: PlantProfileVariables
) derives Encoder.AsObject,
      Decoder

case class PlantProfileVariables(
    lightMmol: Interval[Int],
    lightLux: Interval[Int],
    temperature: Interval[Int],
    humidity: Interval[Int],
    soilMoisture: Interval[Int],
    soilEc: Interval[Int]
) derives Encoder.AsObject,
      Decoder
