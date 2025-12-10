package co.mycelium.ports

case class Interval[A](start: A, end: A)

case class PlantProfile(
    name: String,
    variables: PlantProfileVariables
)

case class PlantProfileVariables(
    lightMmol: Interval[Int],
    lightLux: Interval[Int],
    temperature: Interval[Int],
    humidity: Interval[Int],
    soilMoisture: Interval[Int],
    soilEc: Interval[Int]
)

trait PlantProfiler[F[_]] {
  def getProfilesForPlant(names: List[String]): F[List[PlantProfile]]
}
