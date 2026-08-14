local card = {
    api_version = 1, id = "GVG_094", name = "Jeeves",
    text = "At the end of each player's turn, that player draws until they have 3 cards.",
    set = "GVG", type = "minion", rarity = "rare", cost = 4, attack = 1, health = 4,
    tags = { "mech" },
    triggers = {{
        event = "turn_ended", active_zones = { "board" },
        effect = function(ctx, self, event)
            ctx:set_data(self, "jeeves_player", event.player)
            ctx:continue_with("fill_hand")
        end,
    }},
}

function card.fill_hand(ctx, self)
    local player = ctx:get_data(self, "jeeves_player")
    local missing = 3 - #ctx:hand(player)
    if missing > 0 then ctx:draw(player, missing) end
end

return card
