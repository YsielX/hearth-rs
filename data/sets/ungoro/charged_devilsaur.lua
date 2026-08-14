return {
    api_version = 1, id = "UNG_099", name = "Charged Devilsaur",
    text = "<b>Charge</b>\n<b>Battlecry:</b> Can't attack heroes this turn.",
    set = "UNGORO", type = "minion", rarity = "epic", cost = 7, attack = 7, health = 7,
    tags = { "elemental", "beast" }, keywords = { "charge", "battlecry" },
    on_battlecry = function(ctx, self) ctx:set_data(self, "hero_attack_restricted", 1) end,
    auras = {{
        active_zones = { "board" }, keywords = { "cannot_be_attacked_by_charged_devilsaur" },
        targets = function(ctx, self)
            if ctx:get_data(self, "hero_attack_restricted") ~= 1 then return {} end
            return { ctx:player(ctx:opponent(ctx:controller(self))).hero }
        end,
    }},
    triggers = {{
        event = "turn_ended", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event)
            return event.player == ctx:controller(self) and ctx:get_data(self, "hero_attack_restricted") == 1
        end,
        effect = function(ctx, self) ctx:set_data(self, "hero_attack_restricted", 0) end,
    }},
}
