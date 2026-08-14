return {
    api_version = 1,
    id = "BRM_011",
    name = "Lava Shock",
    text = "Deal $2 damage.\nUnlock your <b>Overloaded</b> Mana Crystals.",
    set = "BRM",
    type = "spell",
    class = "shaman",
    cost = 2,
    rarity = "rare",
    spell_school = "fire",
    target_mode = "required",
    targets = function(ctx, self) return ctx:characters() end,
    on_play = function(ctx, self, target)
        local player = ctx:controller(self)
        ctx:damage(target, 2)
        ctx:clear_overload(player)
    end,
}
