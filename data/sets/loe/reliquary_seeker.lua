local card = {
    api_version = 1,
    id = "LOE_116",
    name = "Reliquary Seeker",
    text = "<b>Battlecry:</b> If you have 6 other minions, gain +4/+4.",
    set = "LOE",
    type = "minion",
    class = "warlock",
    rarity = "rare",
    cost = 1,
    attack = 1,
    health = 1,
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    if #ctx:board(ctx:controller(self)) == 7 then ctx:buff(self, 4, 4) end
end

return card
