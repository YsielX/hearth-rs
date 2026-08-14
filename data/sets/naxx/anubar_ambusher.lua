local card = {
    api_version = 1,
    id = "FP1_026",
    name = "Anub'ar Ambusher",
    text = "<b>Deathrattle:</b> Return a random friendly minion to your hand.",
    set = "NAXX",
    type = "minion",
    class = "rogue",
    rarity = "common",
    cost = 4,
    attack = 5,
    health = 5,
    tags = { "undead" },
    keywords = { "deathrattle" },
}

function card.on_deathrattle(ctx, self)
    local candidates = ctx:friendly_minions(self)
    if #candidates > 0 then ctx:random_entity(candidates, "return_minion") end
end

function card.return_minion(ctx, self, target)
    ctx:move(target, "hand")
end

return card
