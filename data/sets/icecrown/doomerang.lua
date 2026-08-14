local card = {
    api_version = 1, id = "ICC_233", name = "Doomerang",
    text = "Throw your weapon at a minion. It deals its damage, then returns to your hand.",
    set = "ICECROWN", type = "spell", class = "rogue", rarity = "epic", cost = 1,
    target_mode = "required", targets = function(ctx) return ctx:minions() end,
    rules = { can_play = function(ctx, self, current)
        return current and ctx:player(ctx:controller(self)).weapon ~= nil
    end },
}

function card.on_play(ctx, self, target)
    local weapon = ctx:player(ctx:controller(self)).weapon
    if weapon == nil then return end
    ctx:damage_from(weapon, target, ctx:entity(weapon).attack)
    ctx:continue_with_entity("return_doomerang_weapon", weapon)
end

function card.return_doomerang_weapon(ctx, self, weapon)
    if ctx:entity(weapon).zone == "weapon" then ctx:move(weapon, "hand") end
end

return card
