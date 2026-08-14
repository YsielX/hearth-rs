local card = {
    api_version = 1,
    id = "GVG_059",
    name = "Coghammer",
    text = "<b>Battlecry:</b> Give a random friendly minion <b>Divine Shield</b> and <b>Taunt</b>.",
    set = "GVG",
    type = "weapon",
    class = "paladin",
    rarity = "epic",
    cost = 3,
    attack = 2,
    health = 3,
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local candidates = ctx:friendly_minions(self)
    if #candidates > 0 then ctx:random_entity(candidates, "protect_minion") end
end

function card.protect_minion(ctx, self, target)
    ctx:grant_keyword(target, "divine_shield")
    ctx:grant_keyword(target, "taunt")
end

return card
