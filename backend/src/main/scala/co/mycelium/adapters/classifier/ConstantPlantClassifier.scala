package co.mycelium.adapters.classifier

import cats.Applicative
import co.mycelium.ports.PlantClassifier

class ConstantPlantClassifier[F[_]: Applicative] extends PlantClassifier[F] {
  override def classifyPlant(image: fs2.Stream[F, Byte]): F[List[String]] =
    Applicative[F].pure(List("Schefflera"))
}
