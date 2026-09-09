use rand::RngExt;

use crate::{Error, bot::Context};

const BRYCE_INSULTS: &[&str] = &[
    "Bryce smells.",
    "Bryce still hasn't returned that library book from third grade.",
    "Bryce's WiFi password is literally `password123`.",
    "Bryce puts ketchup on everything, even ketchup.",
    "Bryce's ringtone is the Nokia jingle. In 2026.",
    "Bryce thinks a baker's dozen is some kind of scam.",
    "Bryce claps when the plane lands.",
    "Bryce still says 'as seen on TV' unironically.",
    "Bryce has 217 browser tabs open and not one of them is loading.",
    "Bryce microwaves fish in the office break room.",
    "Bryce alphabetizes his sock drawer by color, then gets mad it doesn't work.",
    "Bryce thinks Comic Sans is 'kind of a vibe.'",
    "Bryce's dance moves peaked at the sprinkler.",
    "Bryce once lost a staring contest to a photograph.",
    "Bryce's password hint is just his password.",
    "Bryce calls his Wi-Fi router 'the internet box.'",
    "Bryce still hasn't beaten the tutorial level.",
    "Bryce's home gym is a resistance band he's never opened.",
    "Bryce thinks 'lol' is pronounced 'lull'.",
    "Bryce brings his own tupperware to buffets.",
];

/// Print a random (extremely lame) insult about Bryce
#[poise::command(slash_command)]
pub async fn bryce(ctx: Context<'_>) -> Result<(), Error> {
    // Scoped so the (non-`Send`) RNG is dropped before the `.await` below.
    let insult = {
        let mut rng = rand::rng();
        BRYCE_INSULTS[rng.random_range(0..BRYCE_INSULTS.len())]
    };

    ctx.say(insult).await?;

    Ok(())
}
