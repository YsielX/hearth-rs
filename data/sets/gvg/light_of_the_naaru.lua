local card = {
    api_version = 1,
    id = "GVG_012",
    name = "Light of the Naaru",
    text = "Restore #3 Health. If the target is still damaged, summon a Lightwarden.",
    set = "GVG",
    type = "spell",
    class = "priest",
    rarity = "rare",
    cost = 1,
    spell_school = "holy",
    target_mode = "required",
    targets = function(ctx, self)
        return ctx:characters()
    end,
}

function card.on_play(ctx, self, target)
    ctx:heal(target, 3)
    ctx:continue_with_entity("check_target", target)
end

function card.check_target(ctx, self, target)
    local entity = ctx:entity(target)
    if entity.health < entity.max_health then
        ctx:summon(ctx:controller(self), "EX1_001")
    end
end

return card
