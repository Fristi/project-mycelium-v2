package co.mycelium.adapters.classifier

import cats.effect.Sync
import cats.implicits.*
import co.mycelium.ports.PlantClassifier
import com.sksamuel.scrimage.ImmutableImage
import com.sksamuel.scrimage.nio.JpegWriter
import io.circe.Decoder
import io.circe.parser.decode
import fs2.*
import sttp.ai.openai.OpenAI
import sttp.ai.openai.requests.completions.chat.ChatRequestBody.{
  ChatBody,
  ChatCompletionModel,
  ResponseFormat
}
import sttp.ai.openai.requests.completions.chat.ChatRequestResponseData
import sttp.ai.openai.requests.completions.chat.message.*
import sttp.client4.Backend
import sttp.tapir.Schema
import sttp.tapir.docs.apispec.schema.TapirSchemaToJsonSchema
import sttp.tapir.generic.auto.*

class OpenAiPlantClassifier[F[_]: Sync](apiKey: String, backend: Backend[F]) extends PlantClassifier[F] {
  val openAi = new OpenAI(apiKey)

  private case class PlantClassification(possibleNames: List[String]) derives Decoder

  def resizeImagePipe(targetWidth: Int): Pipe[F, Byte, Byte] =
    in =>
      for {
        // Collect all incoming bytes into a single array
        bytes <- Stream.eval[F, Array[Byte]](in.compile.to(Array))
        // Decode into Scrimage immutable image
        image <- Stream.eval(Sync[F].delay(ImmutableImage.loader().fromBytes(bytes)))
        // Resize it
        resized <- Stream.eval[F, ImmutableImage](
          Sync[F].delay(
            image.scaleToWidth(targetWidth)
          )
        )
        // Encode to bytes (JPEG here—change writer as needed)
        outBytes <- Stream.eval[F, Array[Byte]](
          Sync[F].delay(
            resized.bytes(JpegWriter.Default)
          )
        )
        // Output as fs2 stream of bytes
        out <- Stream.chunk[F, Byte](Chunk.array(outBytes))
      } yield out

  override def classifyPlant(image: Stream[F, Byte]): F[List[String]] = {

    val jsonSchema: Schema[PlantClassification] =
      implicitly[Schema[PlantClassification]]

    val responseFormat =
      ResponseFormat.JsonSchema(
        name = "plantClassification",
        strict = Some(true),
        schema = Some(TapirSchemaToJsonSchema(jsonSchema, markOptionsAsNullable = true)),
        description = Some("Plant classification result.")
      )

    def classifyPlant(base64: String): F[Seq[PlantClassification]] = {
      val requestMessages = Seq(
        Message.UserMessage(content =
          Content.ArrayContent(
            Seq(
              Content.ImageContentPart(Content.ImageUrl(s"data:image/jpg;base64,$base64")),
              Content.TextContentPart("What plant do you see ?")
            )
          )
        )
      )

      val body = ChatBody(
        model = ChatCompletionModel.GPT4oMini,
        messages = requestMessages,
        responseFormat = Some(responseFormat)
      )

      backend.send(openAi.createChatCompletion(body)).map(_.body).flatMap {
        case Left(err)   => Sync[F].raiseError(err)
        case Right(resp) =>
          Sync[F].fromEither(
            resp.choices.traverse(x => decode[PlantClassification](x.message.content))
          )
      }
    }

    for {
      imageBase64 <- image
        .through(resizeImagePipe(264))
        .through(text.base64.encode)
        .reduce(_ + _)
        .compile
        .lastOrError

      classifications <- classifyPlant(imageBase64)

    } yield classifications.flatMap(_.possibleNames).distinct.toList
  }
}
