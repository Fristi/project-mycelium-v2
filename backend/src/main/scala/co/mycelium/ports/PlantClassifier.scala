package co.mycelium.ports

import cats.Applicative
import cats.effect.{Async, IO, Resource}
import co.mycelium.adapters.classifier.{ConstantPlantClassifier, OpenAiPlantClassifier}
import co.mycelium.{ImplementationType, PlantClassifierConfig}
import fs2.Stream
import sttp.client4.httpclient.cats.HttpClientCatsBackend

trait PlantClassifier[F[_]] {
  def classifyPlant(image: Stream[F, Byte]): F[List[String]]
}

object PlantClassifier {
  def fromConfig[F[_]: {Async, Applicative}](
      cfg: PlantClassifierConfig
  ): Resource[F, PlantClassifier[F]] =
    cfg.implementation match {
      case ImplementationType.Production =>
        HttpClientCatsBackend
          .resource[F]()
          .map(b => new OpenAiPlantClassifier[F](cfg.openAiKey.value, b))
      case ImplementationType.Constant => Resource.pure(new ConstantPlantClassifier[F]())
    }
}
