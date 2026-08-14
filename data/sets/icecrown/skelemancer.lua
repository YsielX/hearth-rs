local card = {
    api_version = 1, id = "ICC_019", name = "Skelemancer",
    text = "<b>Deathrattle:</b> If it's your opponent's turn, summon an 8/8 Skeleton.",
    set = "ICECROWN", type = "minion", rarity = "common",
    cost = 5, attack = 2, health = 2, tags = { "undead" }, keywords = { "deathrattle" },
}

function card.on_deathrattle(ctx, self)
    local player = ctx:controller(self)
    if ctx:active_player() ~= player then ctx:summon(player, "ICC_019t") end
end

card.tokens = {{
    id = "ICC_019t", name = "Skeletal Flayer", text = "", set = "ICECROWN",
    type = "minion", collectible = false, cost = 8, attack = 8, health = 8, tags = { "undead" },
}}

return card
