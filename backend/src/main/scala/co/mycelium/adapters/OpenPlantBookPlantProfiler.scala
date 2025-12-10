package co.mycelium.adapters

import cats.effect.Sync
import cats.implicits.*
import co.mycelium.domain.{Interval, PlantProfile, PlantProfileVariables}
import co.mycelium.ports.*
import io.circe.Decoder
import sttp.client4.*
import sttp.client4.circe.*
import sttp.model.StatusCode

class OpenPlantBookPlantProfiler[F[_]: {Sync}](apiKey: String, backend: Backend[F])
    extends PlantProfiler[F] {

  val baseUrl = "https://open.plantbook.io/api/v1/plant"

  private case class PlantBookSearchResult(
      pid: String
  ) derives Decoder

  private case class PlantBookSearchResults(
      results: List[PlantBookSearchResult]
  ) derives Decoder

  private case class PlantBookDetails(
      display_pid: String,
      alias: String,
      category: String,
      max_light_mmol: Int,
      min_light_mmol: Int,
      max_light_lux: Int,
      min_light_lux: Int,
      max_temp: Int,
      min_temp: Int,
      max_env_humid: Int,
      min_env_humid: Int,
      max_soil_moist: Int,
      min_soil_moist: Int,
      max_soil_ec: Int,
      min_soil_ec: Int
  ) derives Decoder

  override def getProfilesForPlant(names: List[String]): F[List[PlantProfile]] = {

    def search(name: String): F[PlantBookSearchResults] = {
      val req =
        basicRequest
          .get(uri"$baseUrl/search?alias=$name")
          .header("Authorization", s"Token $apiKey")
          .response(asJson[PlantBookSearchResults])

      backend.send(req).flatMap(x => Sync[F].fromEither(x.body))
    }

    def details(pid: String): F[Option[PlantBookDetails]] = {
      val req =
        basicRequest
          .get(uri"$baseUrl/detail/$pid/")
          .header("Authorization", s"Token $apiKey")
          .response(asJson[PlantBookDetails])

      backend.send(req).flatMap { x =>
        x.code match {
          case StatusCode.NotFound => Sync[F].pure(None)
          case _                   => Sync[F].fromEither(x.body).map(Some(_))
        }
      }
    }

    def fromOpenPlantBook(p: PlantBookDetails) = PlantProfile(
      p.display_pid,
      PlantProfileVariables(
        Interval(p.min_light_mmol, p.max_light_mmol),
        Interval(p.min_light_lux, p.max_light_lux),
        Interval(p.min_temp, p.max_temp),
        Interval(p.min_env_humid, p.max_env_humid),
        Interval(p.min_soil_moist, p.max_soil_moist),
        Interval(p.min_soil_ec, p.max_soil_ec)
      )
    )

    for {
      results <- names.traverse(search)
      details <- results.flatMap(_.results.map(_.pid)).traverse(details)
    } yield details.flatten.map(fromOpenPlantBook)
  }
}
