package co.mycelium.ports

import cats.Applicative
import cats.effect.{Async, Resource}
import co.mycelium.{ImplementationType, PlantProfilerConfig}
import co.mycelium.adapters.{ConstantPlantProfiler, OpenPlantBookPlantProfiler}
import co.mycelium.domain.PlantProfile
import sttp.client4.httpclient.cats.HttpClientCatsBackend

trait PlantProfiler[F[_]] {
  def getProfilesForPlant(names: List[String]): F[List[PlantProfile]]
}

object PlantProfiler {
  def fromConfig[F[_]: {Async, Applicative}](
      cfg: PlantProfilerConfig
  ): Resource[F, PlantProfiler[F]] =
    cfg.implementation match {
      case ImplementationType.Production =>
        HttpClientCatsBackend
          .resource[F]()
          .map(b => new OpenPlantBookPlantProfiler[F](cfg.openPlantBookKey.value, b))
      case ImplementationType.Constant => Resource.pure(new ConstantPlantProfiler[F]())
    }
}
