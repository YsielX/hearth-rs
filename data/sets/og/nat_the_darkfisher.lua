local card = {
    api_version = 1, id = "OG_338", name = "Nat, the Darkfisher",
    text = "At the start of your opponent's turn, they have a 50% chance to draw an extra card.",
    set = "OG", type = "minion", rarity = "legendary", cost = 2, attack = 2, health = 4,
    triggers = {{
        event = "turn_started", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event) return event.player == ctx:opponent(ctx:controller(self)) end,
        effect = function(ctx, self) ctx:random_value({ 0, 1 }, "resolve_darkfisher") end,
    }},
}
function card.resolve_darkfisher(ctx, self, result)
    if result == 1 then ctx:draw(ctx:opponent(ctx:controller(self)), 1) end
end
return card
