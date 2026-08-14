local card = {
    api_version = 1, id = "CFM_800", name = "Getaway Kodo",
    text = "<b>Secret:</b> When a friendly minion dies, return it to your hand.",
    set = "GANGS", type = "spell", class = "paladin", rarity = "rare",
    cost = 1, keywords = { "secret" },
    triggers = {{
        event = "entity_died", timing = "after", active_zones = { "secret" },
        condition = function(ctx, self, event)
            return event.player == ctx:controller(self) and ctx:entity(event.entity).type == "minion"
                and #ctx:hand(event.player) < 10
        end,
        effect = function(ctx, self, event)
            ctx:continue_with_value("return_getaway_kodo", event.entity)
        end,
    }},
}

function card.return_getaway_kodo(ctx, self, entity)
    if ctx:get_data(self, "triggered") == 1 then return end
    ctx:set_data(self, "triggered", 1)
    ctx:reveal_secret(self)
    ctx:move(entity, "hand")
end

return card
