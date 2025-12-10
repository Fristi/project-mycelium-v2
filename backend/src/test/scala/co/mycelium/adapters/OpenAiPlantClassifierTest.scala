package co.mycelium.adapters

import cats.effect.{IO, Resource}
import cats.implicits.*
import co.mycelium.adapters.OpenAiPlantClassifier
import co.mycelium.ports.PlantClassifier
import sttp.client4.httpclient.cats.HttpClientCatsBackend
import weaver.*

object OpenAiPlantClassifierTest extends IOSuite {
  override type Res = PlantClassifier[IO]

  override def sharedResource: Resource[IO, PlantClassifier[IO]] =
    HttpClientCatsBackend
      .resource[IO]()
      .map(backend => new OpenAiPlantClassifier[IO](System.getenv("OPENAI_KEY"), backend))

  test("should classify plant") { client =>
    for {
      _ <- client.classifyPlant(
        fs2.io.readInputStream(IO.delay(getClass.getResourceAsStream("/plant.png")), 1024)
      )
    } yield expect.same(1, 1)
  }
}
