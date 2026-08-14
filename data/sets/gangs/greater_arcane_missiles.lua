local card = {
    api_version = 1,
    id = "CFM_623",
    name = "Greater Arcane Missiles",
    text = "Shoot three missiles at random enemies that deal $3 damage each.",
    set = "GANGS",
    type = "spell",
    class = "mage",
    rarity = "epic",
    spell_school = "arcane",
    cost = 7,
}

function card.on_play(ctx, self)
    ctx:set_data(self, "missiles_remaining", 3)
    ctx:continue_with("choose_missile_target")
end

function card.choose_missile_target(ctx, self)
    if ctx:get_data(self, "missiles_remaining") <= 0 then return end
    local candidates = ctx:enemy_characters(self)
    if #candidates > 0 then ctx:random_entity(candidates, "fire_missile") end
end

function card.fire_missile(ctx, self, target)
    ctx:set_data(self, "missiles_remaining", ctx:get_data(self, "missiles_remaining") - 1)
    ctx:damage(target, 3)
    ctx:continue_with("choose_missile_target")
end

return card
