local card = {
    api_version = 1, id = "CFM_663", name = "Kabal Trafficker",
    text = "[x]At the end of your turn,\nadd a random Demon\nto your hand.",
    set = "GANGS", type = "minion", class = "warlock", rarity = "epic",
    cost = 6, attack = 6, health = 6,
    triggers = {{
        event = "turn_ended", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event) return event.player == ctx:controller(self) end,
        effect = function(ctx, self)
            local pool = {}
            for _, card_id in ipairs(ctx:collectible_cards()) do
                local definition = ctx:card_definition(card_id)
                if definition.type == "minion" then
                    for _, tag in ipairs(definition.tags or {}) do
                        if tag == "demon" or tag == "all" then pool[#pool + 1] = card_id break end
                    end
                end
            end
            if #pool > 0 then ctx:random_value(pool, "receive_demon") end
        end,
    }},
}
function card.receive_demon(ctx, self, card_id) ctx:give_card(ctx:controller(self), card_id) end
return card
