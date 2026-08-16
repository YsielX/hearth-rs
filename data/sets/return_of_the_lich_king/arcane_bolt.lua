return {
    api_version = 1,
    id = "RLK_843",
    name = "Arcane Bolt",
    text = "Deal $2 damage. <b>Manathirst (8):</b> Deal $3 damage instead.",
    set = "RETURN_OF_THE_LICH_KING",
    type = "spell",
    class = "mage",
    cost = 1,
    keywords = { "manathirst" },
    keyword_params = { manathirst = 8 },
    target_mode = "required",
    targets = function(ctx, self) return ctx:characters() end,
    on_play = function(ctx, self, target)
        if ctx:player(ctx:controller(self)).max_mana < 8 then
            cardlib.effects.damage(ctx, target, 2)
        end
    end,
    on_manathirst = function(ctx, self, target) cardlib.effects.damage(ctx, target, 3) end,
}
