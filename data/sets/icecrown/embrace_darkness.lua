local card = {
    api_version = 1, id = "ICC_849", name = "Embrace Darkness",
    text = "[x]Choose an enemy minion.\nAt the start of your turn,\ngain control of it.",
    set = "ICECROWN", type = "spell", class = "priest", rarity = "epic",
    spell_school = "shadow", cost = 6, target_mode = "required",
    targets = function(ctx, self)
        local targets = {}
        for _, entity in ipairs(ctx:enemy_characters(self)) do
            if ctx:entity(entity).type == "minion" then targets[#targets + 1] = entity end
        end
        return targets
    end,
}

function card.on_play(ctx, self, target)
    ctx:set_data(target, "embrace_darkness_owner", ctx:controller(self) + 1)
    ctx:attach_script(target, "ICC_849")
end

card.triggers = {{
    event = "turn_started", timing = "after", active_zones = { "board" },
    condition = function(ctx, self, event)
        local owner = ctx:get_data(self, "embrace_darkness_owner") - 1
        return owner >= 0 and event.player == owner and ctx:controller(self) ~= owner
    end,
    effect = function(ctx, self)
        local owner = ctx:get_data(self, "embrace_darkness_owner") - 1
        ctx:set_data(self, "embrace_darkness_owner", 0)
        ctx:change_controller(self, owner)
    end,
}}

return card
