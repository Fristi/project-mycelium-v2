package co.mycelium.db

import cats.effect.IO
import cats.effect.kernel.Resource
import cats.implicits.*
import co.mycelium.adapters.db.*
import co.mycelium.domain.*
import doobie.weaver.*
import weaver.*
import doobie.*
import doobie.implicits.*

import java.time.Instant
import java.util.UUID

object DoobieStationProfileRepositoryTest extends IOSuite with IOChecker {

  override type Res = Transactor[IO]

  val stationRepo            = DoobieStationRepository
  val profileRepo            = DoobieStationProfileRepository
  val now                    = Instant.now()
  val plantProfileVariables1 = PlantProfileVariables(
    Interval(1500, 3400),
    Interval(1000, 18000),
    Interval(6, 32),
    Interval(30, 85),
    Interval(15, 60),
    Interval(350, 2000)
  )
  val plantProfileVariables2 = PlantProfileVariables(
    Interval(1300, 3200),
    Interval(1100, 20000),
    Interval(7, 30),
    Interval(31, 86),
    Interval(16, 50),
    Interval(340, 1000)
  )
  val plantProfile1 = PlantProfile("Plant", plantProfileVariables1)
  val plantProfile2 = PlantProfile("Plant 2", plantProfileVariables2)

  override def sharedResource: Resource[IO, Res] =
    DoobieResource.setup

  private val mac1 = "00:00:00:00:00:00"
  private val mac2 = "00:00:00:00:00:01"
  test("insert should make an entry") { implicit tx =>
    val insert  = StationInsert(mac1, "Unnamed")
    val userId  = "some-user-id"
    val station = insert.toStation(UUID.randomUUID(), now, userId)

    val program = for {
      id1      <- stationRepo.insert(station, now)
      _        <- profileRepo.upsert(plantProfile1, id1)
      profiles <- profileRepo.getPlantProfilesByUserId(userId)
    } yield expect.eql(profiles, List(StationPlantProfile(id1, mac1, plantProfile1)))

    program.transact(tx)
  }

  test("should up upsert an entry") { implicit tx =>
    val insert  = StationInsert(mac1, "Unnamed")
    val userId  = "some-user-id"
    val station = insert.toStation(UUID.randomUUID(), now, userId)

    val program = for {
      id1      <- stationRepo.insert(station, now)
      _        <- profileRepo.upsert(plantProfile1, id1)
      _        <- profileRepo.upsert(plantProfile2, id1)
      profiles <- profileRepo.getPlantProfilesByUserId(userId)
    } yield expect.eql(profiles, List(StationPlantProfile(id1, mac1, plantProfile2)))

    program.transact(tx)
  }

  test("should yield multiple profiles") { implicit tx =>
    val userId               = "some-user-id"
    def station(mac: String) =
      StationInsert(mac, "Unnamed").toStation(UUID.randomUUID(), now, userId)

    val program = for {
      id1      <- stationRepo.insert(station(mac1), now)
      id2      <- stationRepo.insert(station(mac2), now)
      _        <- profileRepo.upsert(plantProfile1, id1)
      _        <- profileRepo.upsert(plantProfile2, id2)
      profiles <- profileRepo.getPlantProfilesByUserId(userId)
    } yield expect.eql(
      profiles,
      List(
        StationPlantProfile(id1, mac1, plantProfile1),
        StationPlantProfile(id2, mac2, plantProfile2)
      )
    )

    program.transact(tx)
  }
}
