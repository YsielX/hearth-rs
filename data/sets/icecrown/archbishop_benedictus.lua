local card = {
    api_version = 1, id = "ICC_215", name = "Archbishop Benedictus",
    text = "<b>Battlecry:</b> Shuffle a copy of your opponent's deck into your deck.",
    set = "ICECROWN", type = "minion", class = "priest", rarity = "legendary",
    cost = 7, attack = 4, health = 6, keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local player = ctx:controller(self)
    local opponent_deck = ctx:deck(ctx:opponent(player))
    for _, entity in ipairs(opponent_deck) do
        ctx:shuffle_copy_into_deck(player, entity)
    end
end

return card
