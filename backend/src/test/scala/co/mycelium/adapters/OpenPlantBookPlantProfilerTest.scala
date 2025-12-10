package co.mycelium.adapters

import cats.effect.{IO, Resource}
import co.mycelium.ports.{Interval, PlantProfile, PlantProfileVariables, PlantProfiler}
import sttp.client4.httpclient.cats.HttpClientCatsBackend
import weaver.IOSuite

object OpenPlantBookPlantProfilerTest extends IOSuite {
  override type Res = PlantProfiler[IO]

  override def sharedResource: Resource[IO, PlantProfiler[IO]] =
    HttpClientCatsBackend
      .resource[IO]()
      .map(backend =>
        new OpenPlantBookPlantProfiler[IO](System.getenv("OPENPLANTBOOK_KEY"), backend)
      )

  test("should get profiles") { client =>
    for {
      results <- client.getProfilesForPlant(List("Schefflera"))
      expected = List(
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
    } yield expect.same(results, expected)
  }
}
