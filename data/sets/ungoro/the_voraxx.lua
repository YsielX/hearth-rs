local card = {
    api_version = 1, id = "UNG_843", name = "The Voraxx",
    text = "[x]After you cast a spell on\nthis minion, summon a\n1/1 Plant and cast\nanother copy on it.",
    set = "UNGORO", type = "minion", rarity = "legendary", cost = 3, attack = 3, health = 4,
    triggers = {{
        event = "spell_cast", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event)
            return event.player == ctx:controller(self) and event.player_cast and event.target == self
        end,
        effect = function(ctx, self, event)
            ctx:set_data(self, "voraxx_spell", event.entity)
            local floor = 0
            for _, entity in ipairs(ctx:minions()) do if entity > floor then floor = entity end end
            ctx:set_data(self, "voraxx_floor", floor)
            ctx:summon(ctx:controller(self), "UNG_999t2t1")
            ctx:continue_with("copy_spell_to_plant")
        end,
    }},
}
function card.copy_spell_to_plant(ctx, self)
    local plant = nil
    local floor = ctx:get_data(self, "voraxx_floor")
    for _, entity in ipairs(ctx:friendly_minions(self)) do
        if entity > floor and ctx:entity(entity).card_id == "UNG_999t2t1" and (plant == nil or entity > plant) then plant = entity end
    end
    local spell = ctx:get_data(self, "voraxx_spell")
    if plant and spell and spell ~= 0 then
        ctx:cast_spell(ctx:controller(self), ctx:entity(spell).card_id, {
            target = plant,
            skip_if_invalid = true,
        })
    end
end
return card
