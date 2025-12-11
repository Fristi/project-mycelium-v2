package co.mycelium.adapters.profiler

import cats.Applicative
import co.mycelium.domain.{Interval, PlantProfile, PlantProfileVariables}
import co.mycelium.ports.PlantProfiler

case class ConstantPlantProfiler[F[_]: Applicative]() extends PlantProfiler[F] {
  override def getProfilesForPlant(names: List[String]): F[List[PlantProfile]] =
    Applicative[F].pure(
      List(
        PlantProfile(
          "Schefflera elegantissima",
          PlantProfileVariables(
            Interval(2500, 4500),
            Interval(3700, 30000),
            Interval(5, 32),
            Interval(30, 80),
            Interval(15, 60),
            Interval(350, 2000)
          )
        ),
        PlantProfile(
          "Schefflera impressa",
          PlantProfileVariables(
            Interval(1500, 3400),
            Interval(1000, 18000),
            Interval(6, 32),
            Interval(30, 85),
            Interval(15, 60),
            Interval(350, 2000)
          )
        ),
        PlantProfile(
          "Schefflera octophylla",
          PlantProfileVariables(
            Interval(1500, 3400),
            Interval(1000, 18000),
            Interval(6, 32),
            Interval(30, 85),
            Interval(15, 60),
            Interval(350, 2000)
          )
        ),
        PlantProfile(
          "Schefflera taiwaniana",
          PlantProfileVariables(
            Interval(1500, 3400),
            Interval(1000, 18000),
            Interval(6, 32),
            Interval(30, 85),
            Interval(15, 60),
            Interval(350, 2000)
          )
        ),
        PlantProfile(
          "Schefflera actinophylla",
          PlantProfileVariables(
            Interval(1500, 4600),
            Interval(1200, 30000),
            Interval(8, 35),
            Interval(30, 85),
            Interval(15, 60),
            Interval(350, 2000)
          )
        ),
        PlantProfile(
          "Schefflera arboricola",
          PlantProfileVariables(
            Interval(2000, 4000),
            Interval(3700, 20000),
            Interval(10, 32),
            Interval(30, 80),
            Interval(15, 60),
            Interval(350, 2000)
          )
        )
      )
    )
}
