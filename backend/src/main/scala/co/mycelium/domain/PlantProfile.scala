package co.mycelium.domain

import cats.kernel.Eq
import cats.derived.*
import io.circe.{Decoder, Encoder}

import java.util.UUID

case class Interval[A](start: A, end: A)

object Interval {
  given [A: Encoder]: Encoder.AsObject[Interval[A]] =
    Encoder.AsObject.derived

  given [A: Decoder]: Decoder[Interval[A]] =
    Decoder.derived

  given [A: Eq]: Eq[Interval[A]] = new Eq[Interval[A]] {
    override def eqv(x: Interval[A], y: Interval[A]): Boolean = x.start == y.start && x.end == y.end
  }
}

case class StationPlantProfile(
    stationId: UUID,
    mac: String,
    profile: PlantProfile
) derives Encoder.AsObject,
      Decoder,
      Eq

case class PlantProfile(
    name: String,
    variables: PlantProfileVariables
) derives Encoder.AsObject,
      Decoder,
      Eq

case class PlantProfileVariables(
    lightMmol: Interval[Int],
    lightLux: Interval[Int],
    temperature: Interval[Int],
    humidity: Interval[Int],
    soilMoisture: Interval[Int],
    soilEc: Interval[Int]
) derives Encoder.AsObject,
      Decoder
