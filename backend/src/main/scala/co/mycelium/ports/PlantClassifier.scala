package co.mycelium.ports

import fs2.Stream

trait PlantClassifier[F[_]] {
  def classifyPlant(image: Stream[F, Byte]): F[List[String]]
}
