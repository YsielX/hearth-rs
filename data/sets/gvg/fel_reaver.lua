local card = {
    api_version = 1, id = "GVG_016", name = "Fel Reaver",
    text = "Whenever your opponent plays a card, remove the top 3 cards of your deck.",
    set = "GVG", type = "minion", rarity = "epic", cost = 5, attack = 8, health = 8,
    tags = { "mech" },
    triggers = {{
        event = "card_played", active_zones = { "board" },
        condition = function(ctx, self, event) return event.player ~= ctx:controller(self) end,
        effect = function(ctx) ctx:continue_with("remove_top_three") end,
    }},
}

function card.remove_top_three(ctx, self)
    local deck = ctx:deck(ctx:controller(self))
    for index = 1, math.min(3, #deck) do ctx:move(deck[index], "removed") end
end

return card
