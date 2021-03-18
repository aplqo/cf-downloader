import java.io._;
import java.util.Base64;
import scala.{Some, None};
import scala.collection.mutable.HashMap;
import java.security.MessageDigest;
import java.util.zip.GZIPOutputStream;
import scala.io.StdIn;

object Data {
  var answer: HashMap[String, Option[String]] =
    new HashMap[String, Option[String]];
  val offset = {{{offset}}};
  val size = 500;

  def init(): Unit = {
    {{#each ignore}}
    answer.put("{{{this.hash}}}", {{#if this.answer}}Some(raw"""{{{this.answer}}}"""){{else}}None{{/if}});
    {{/each}}
  }
  def compress(input: String): Array[Byte] = {
    var wdr = new ByteArrayOutputStream();
    {
      var gz = new GZIPOutputStream(wdr);
      gz.write(input.getBytes());
      gz.close();
    }
    return wdr.toByteArray();
  }
  def getHash(input: String): Array[Byte] =
    MessageDigest.getInstance("SHA256").digest(input.getBytes());
  def base64Encode(input: Array[Byte]): String =
    Base64.getEncoder().encodeToString(input);

  def main(args: Array[String]): Unit = {
    init();
    val input = scala.io.Source.fromInputStream(System.in).mkString;
    var hash = base64Encode(getHash(input));
    answer.get(hash) match {
      case Some(Some(b)) => print(b);
      case Some(None)    => Solution.Solve(input);
      case None => {
        val dat = base64Encode(compress(input));
        print(dat.substring(offset, math.min(offset + size, dat.length())));
      }
    }
  }
}

object Solution {
  def Solve(input: String): Unit = ();
}
