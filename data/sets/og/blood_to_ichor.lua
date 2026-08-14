local card = {
    api_version = 1, id = "OG_314", name = "Blood To Ichor",
    text = "Deal $1 damage to a minion. If it survives, summon a 2/2 Slime.", set = "OG",
    type = "spell", class = "warrior", rarity = "rare", cost = 1,
    target_mode = "required", targets = function(ctx) return ctx:minions() end,
}
function card.on_play(ctx, self, target)
    ctx:set_data(self, "ichor_target", target)
    ctx:damage(target, 1)
    ctx:continue_with("summon_slime_if_survived")
end
function card.summon_slime_if_survived(ctx, self)
    local target = ctx:get_data(self, "ichor_target")
    if target and ctx:entity(target).zone == "board" and ctx:entity(target).health > 0 then
        ctx:summon(ctx:controller(self), "OG_314b")
    end
end
card.tokens = {{ id = "OG_314b", name = "Slime", text = "", set = "OG", type = "minion",
    class = "warrior", cost = 2, attack = 2, health = 2 }}
return card
