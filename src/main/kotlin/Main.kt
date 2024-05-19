import com.github.ajalt.clikt.completion.completionOption
import com.github.ajalt.clikt.core.context
import com.github.ajalt.clikt.core.subcommands
import com.github.ajalt.clikt.output.CliktHelpFormatter
import ru.tesserakt.kodept.cli.Interpret
import ru.tesserakt.kodept.cli.Kodept
import ru.tesserakt.kodept.cli.Parse
import ru.tesserakt.kodept.cli.Typecheck

fun main(args: Array<String>) = Kodept()
    .context { helpFormatter = CliktHelpFormatter(showDefaultValues = true, showRequiredTag = true) }
    .completionOption(hidden = true)
    .subcommands(Parse().subcommands(Typecheck(), Interpret()))
    .main(args)