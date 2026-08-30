local card = {
    api_version = 1,
    id = "EX1_316",
    name = "Power Overwhelming",
    text = "Give a friendly minion +4/+4 until end of turn. Then, it dies. Horribly.",
    set = "EXPERT1",
    type = "spell",
    class = "warlock",
    rarity = "common",
    spell_school = "shadow",
    cost = 1,
    target_mode = "required",
    targets = function(ctx, self)
        return ctx:friendly_minions(self)
    end,
    on_play = function(ctx, self, target)
        cardlib.effects.buff_until_end_of_turn(ctx, target, 4, 4)
        ctx:attach_script(target, "EX1_316")
    end,
    triggers = { {
        event = "turn_ended",
        timing = "after",
        active_zones = { "board" },
        effect = function(ctx, self)
            cardlib.effects.destroy(ctx, self)
        end,
    } },
}

return card
