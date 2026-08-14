local card = {
    api_version = 1,
    id = "TLC_519",
    name = "Ambush Predators",
    text = "Summon a 1/1 Spitter with <b>Stealth</b>\nand <b>Poisonous</b>.\n<b>Kindred:</b> Do it again.",
    set = "THE_LOST_CITY",
    type = "spell",
    class = "rogue",
    cost = 3,
    spell_school = "shadow",
    keywords = { "kindred" },
}

function card.on_play(ctx, self)
    ctx:summon(ctx:controller(self), "TLC_519t")
end

function card.on_kindred(ctx, self)
    ctx:summon(ctx:controller(self), "TLC_519t")
end

card.tokens = {
    {
        id = "TLC_519t", name = "Venomous Spitter",
        text = "<b>Stealth</b>\n <b>Poisonous</b>",
        set = "THE_LOST_CITY", type = "minion", class = "rogue",
        cost = 2, attack = 1, health = 1, tags = { "beast" },
        keywords = { "stealth", "poisonous" },
    },
}

return card
