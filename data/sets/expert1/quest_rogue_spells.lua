local function friendly_minions(ctx, self)
    return ctx:friendly_minions(self)
end

return {
    api_version = 1,
    id = "EX1_145",
    name = "Preparation",
    text = "The next spell you cast this turn costs (2) less.",
    set = "EXPERT1",
    type = "spell",
    class = "rogue",
    cost = 0,
    on_play = function(ctx, self) ctx:set_data(self, "active", 1) end,
    auras = {
        {
            active_zones = { "graveyard" },
            cost = function(ctx, self)
                if ctx:get_data(self, "active") == 1 then return -2 end
                return 0
            end,
            targets = function(ctx, self)
                local result = {}
                if ctx:get_data(self, "active") == 0 then return result end
                for _, entity in ipairs(ctx:hand(ctx:controller(self))) do
                    if ctx:entity(entity).type == "spell" then result[#result + 1] = entity end
                end
                return result
            end,
        },
    },
    triggers = {
        {
            event = "spell_cast",
            timing = "after",
            active_zones = { "graveyard" },
            condition = function(ctx, self, event)
                return ctx:get_data(self, "active") == 1
                    and event.player == ctx:controller(self)
                    and event.entity ~= self
            end,
            effect = function(ctx, self) ctx:set_data(self, "active", 0) end,
        },
        {
            event = "card_countered",
            timing = "after",
            active_zones = { "graveyard" },
            condition = function(ctx, self, event)
                return ctx:get_data(self, "active") == 1
                    and event.player == ctx:controller(self)
                    and ctx:entity(event.entity).type == "spell"
            end,
            effect = function(ctx, self) ctx:set_data(self, "active", 0) end,
        },
        {
            event = "turn_ended",
            timing = "after",
            active_zones = { "graveyard" },
            condition = function(ctx, self, event)
                return ctx:get_data(self, "active") == 1
                    and event.player == ctx:controller(self)
            end,
            effect = function(ctx, self) ctx:set_data(self, "active", 0) end,
        },
    },
    tokens = {
        {
            id = "EX1_144",
            name = "Shadowstep",
            text = "Return a friendly minion to your hand. It costs (2) less.",
            set = "EXPERT1",
            type = "spell",
            class = "rogue",
            collectible = true,
            cost = 0,
            target_mode = "required",
            targets = friendly_minions,
            on_play = function(ctx, self, target)
                ctx:move(target, "hand")
                ctx:modify(target, { stat = "cost", operation = "add", value = -2 })
            end,
        },
        {
            id = "EX1_124",
            name = "Eviscerate",
            text = "Deal $2 damage. <b>Combo:</b> Deal $4 damage instead.",
            set = "EXPERT1",
            type = "spell",
            class = "rogue",
            collectible = true,
            cost = 2,
            target_mode = "required",
            keywords = { "combo" },
            targets = function(ctx, self) return ctx:enemy_characters(self) end,
            on_play = function(ctx, self, target)
                if not ctx:combo_active(self) then ctx:damage(target, 2) end
            end,
            on_combo = function(ctx, self, target) ctx:damage(target, 4) end,
        },
    },
}
