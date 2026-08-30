local card = {
    api_version = 1, id = "ICC_850", name = "Shadowblade",
    text = "<b>Battlecry:</b> Your hero is <b>Immune</b> this turn.", set = "ICECROWN",
    type = "weapon", class = "rogue", rarity = "rare", cost = 3, attack = 3, health = 2,
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    cardlib.effects.grant_keyword_until_end_of_turn(ctx, ctx:player(ctx:controller(self)).hero, "immune")
end

return card
