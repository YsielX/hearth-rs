local card = {
    api_version = 1, id = "ICC_026", name = "Grim Necromancer",
    text = "<b>Battlecry:</b> Summon two 1/1 Skeletons.",
    set = "ICECROWN", type = "minion", rarity = "common",
    cost = 4, attack = 2, health = 4, tags = { "undead" }, keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local player = ctx:controller(self)
    ctx:summon(player, "ICC_026t")
    ctx:summon(player, "ICC_026t")
end

card.tokens = {{
    id = "ICC_026t", name = "Skeleton", text = "", set = "ICECROWN",
    type = "minion", collectible = false, cost = 1, attack = 1, health = 1, tags = { "undead" },
}}

return card
