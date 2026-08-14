local card = {
    api_version = 1,
    id = "ICC_827",
    name = "Valeera the Hollow",
    text = "<b>Battlecry:</b> Gain <b>Stealth</b> until your next turn.",
    set = "ICECROWN",
    type = "hero",
    class = "rogue",
    rarity = "legendary",
    cost = 9,
    health = 30,
    armor = 5,
    hero_power = "ICC_827p",
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    ctx:grant_keyword_until_next_turn(self, "stealth")
end

return card
