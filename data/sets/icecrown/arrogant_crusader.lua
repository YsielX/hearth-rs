local card = {
    api_version = 1, id = "ICC_034", name = "Arrogant Crusader",
    text = "<b>Deathrattle:</b> If it's your opponent's turn, summon a 2/2 Ghoul.",
    set = "ICECROWN", type = "minion", class = "paladin", rarity = "rare",
    cost = 4, attack = 5, health = 2, tags = { "undead" }, keywords = { "deathrattle" },
}

function card.on_deathrattle(ctx, self, position)
    local player = ctx:controller(self)
    if ctx:active_player() ~= player then ctx:summon_at(player, "ICC_900t", position) end
end

return card
