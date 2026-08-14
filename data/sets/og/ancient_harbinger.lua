local card = {
    api_version = 1, id = "OG_290", name = "Ancient Harbinger",
    text = "At the start of your turn, put a 10-Cost minion from your deck into your hand.", set = "OG",
    type = "minion", rarity = "epic", cost = 6, attack = 4, health = 6,
    triggers = {{
        event = "turn_started", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event) return event.player == ctx:controller(self) end,
        effect = function(ctx, self)
            local candidates = {}
            for _, entity in ipairs(ctx:deck(ctx:controller(self))) do
                local card_in_deck = ctx:entity(entity)
                if card_in_deck.type == "minion" and card_in_deck.cost == 10 then
                    candidates[#candidates + 1] = entity
                end
            end
            if #candidates > 0 then ctx:random_entity(candidates, "put_harbinger_minion_in_hand") end
        end,
    }},
}
function card.put_harbinger_minion_in_hand(ctx, self, target) ctx:move(target, "hand") end
return card
