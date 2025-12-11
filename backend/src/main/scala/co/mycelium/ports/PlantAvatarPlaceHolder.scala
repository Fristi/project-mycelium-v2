package co.mycelium.ports

import cats.effect.Sync
import fs2.Stream

trait PlantAvatarPlaceHolder[F[_]] {
  def get: Stream[F, Byte]
}

class ResourcePlantAvatarPlaceHolder[F[_]: {Sync}] extends PlantAvatarPlaceHolder[F] {
  override def get: Stream[F, Byte] =
    fs2.io
      .readInputStream[F](
        Sync[F].pure(getClass.getResourceAsStream("/placeholder.png")),
        1024
      )
}
