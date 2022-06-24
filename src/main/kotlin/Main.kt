import com.github.ajalt.clikt.core.subcommands
import ru.tesserakt.kodept.cli.Interpret
import ru.tesserakt.kodept.cli.Kodept
import ru.tesserakt.kodept.cli.Parse
import ru.tesserakt.kodept.cli.Typecheck

fun main(args: Array<String>) = Kodept()
    .subcommands(Parse().subcommands(Typecheck(), Interpret()))
    .main(args)