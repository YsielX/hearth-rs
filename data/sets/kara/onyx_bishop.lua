local card = {
    api_version = 1,
    id = "KAR_204",
    name = "Onyx Bishop",
    text = "<b>Battlecry:</b> Summon a random friendly minion\nthat died this game.",
    set = "KARA",
    type = "minion",
    class = "priest",
    rarity = "rare",
    cost = 4,
    attack = 3,
    health = 4,
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local player = ctx:controller(self)
    if #ctx:board(player) >= 7 then return end
    local pool = ctx:minions_died(player)
    if #pool > 0 then ctx:random_value(pool, "summon_dead_minion") end
end

function card.summon_dead_minion(ctx, self, card_id)
    ctx:summon(ctx:controller(self), card_id)
end

return card
